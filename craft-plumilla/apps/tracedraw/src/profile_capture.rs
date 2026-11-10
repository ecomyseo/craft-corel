//! CPU inspection of actual egui meshes and texture updates for private QA.
//! This is deliberately labelled separately from a native-window GPU capture.

use egui::{ColorImage, TextureId, epaint::{ImageData, Primitive, Vertex}};
use std::{collections::HashMap, path::Path};

#[derive(Default)]
pub(crate) struct Capture { textures: HashMap<TextureId, ColorImage> }

impl Capture {
    pub(crate) fn consume(&mut self, output: &mut egui::FullOutput) -> Result<(), String> {
        let result = self.update(output);
        output.textures_delta.clear();
        result
    }

    pub(crate) fn update(&mut self, output: &egui::FullOutput) -> Result<(), String> {
        for (id, patches) in &output.textures_delta.set {
            for delta in patches {
                let ImageData::Color(image) = &delta.image;
                if image.size[0].checked_mul(image.size[1]).is_none_or(|n| n > 64 * 1024 * 1024) {
                    return Err("diagnostic texture exceeds budget".into());
                }
                if let Some([x,y]) = delta.pos {
                    let target = self.textures.get_mut(id).ok_or("partial diagnostic texture has no base")?;
                    if x.checked_add(image.size[0]).is_none_or(|right|right > target.size[0]) ||
                        y.checked_add(image.size[1]).is_none_or(|bottom|bottom > target.size[1]) {
                        return Err("diagnostic texture patch outside base".into());
                    }
                    for row in 0..image.size[1] {
                        let at=(y+row)*target.size[0]+x;
                        let source=image.pixels.get(row*image.size[0]..(row+1)*image.size[0]).ok_or("short diagnostic patch")?;
                        target.pixels.get_mut(at..at+image.size[0]).ok_or("short diagnostic texture")?.copy_from_slice(source);
                    }
                } else { self.textures.insert(*id, image.as_ref().clone()); }
            }
        }
        Ok(())
    }

    pub(crate) fn save(&self, ctx: &egui::Context, output: &egui::FullOutput, size: [u32;2], path: &Path) -> Result<(), String> {
        if (output.pixels_per_point-1.0).abs() > 1e-6 { return Err("diagnostic capture requires scale one".into()); }
        if u64::from(size[0])*u64::from(size[1]) > 4_096_000 { return Err("diagnostic image exceeds budget".into()); }
        let mut image=image::RgbaImage::from_pixel(size[0],size[1],image::Rgba([35,35,35,255]));
        for clipped in ctx.tessellate(output.shapes.clone(), output.pixels_per_point) {
            let Primitive::Mesh(mesh)=clipped.primitive else { return Err("diagnostic capture cannot rasterise a GPU callback".into()); };
            let texture=self.textures.get(&mesh.texture_id).ok_or("diagnostic mesh texture missing")?;
            for index in mesh.indices.chunks_exact(3) {
                let vertices=[
                    *mesh.vertices.get(index[0] as usize).ok_or("mesh vertex outside buffer")?,
                    *mesh.vertices.get(index[1] as usize).ok_or("mesh vertex outside buffer")?,
                    *mesh.vertices.get(index[2] as usize).ok_or("mesh vertex outside buffer")?,
                ];
                triangle(&mut image, texture, clipped.clip_rect, vertices);
            }
        }
        image.save(path).map_err(|e|e.to_string())
    }
}

fn triangle(image:&mut image::RgbaImage, texture:&ColorImage, clip:egui::Rect, v:[Vertex;3]) {
    let [a,b,c]=v.map(|v|v.pos);
    let denominator=(b.y-c.y)*(a.x-c.x)+(c.x-b.x)*(a.y-c.y);
    if !denominator.is_finite() || denominator.abs()<1e-8 { return; }
    let left=a.x.min(b.x).min(c.x).max(clip.left()).floor().max(0.0) as u32;
    let right=a.x.max(b.x).max(c.x).min(clip.right()).ceil().clamp(0.0,image.width() as f32) as u32;
    let top=a.y.min(b.y).min(c.y).max(clip.top()).floor().max(0.0) as u32;
    let bottom=a.y.max(b.y).max(c.y).min(clip.bottom()).ceil().clamp(0.0,image.height() as f32) as u32;
    for y in top..bottom {
        for x in left..right {
            let p=egui::pos2(x as f32+0.5,y as f32+0.5);
            if !clip.contains(p) {continue;}
            let w0=((b.y-c.y)*(p.x-c.x)+(c.x-b.x)*(p.y-c.y))/denominator;
            let w1=((c.y-a.y)*(p.x-c.x)+(a.x-c.x)*(p.y-c.y))/denominator;
            let weights=[w0,w1,1.0-w0-w1];
            if weights.iter().any(|w| !w.is_finite() || *w < -1e-6) {continue;}
            let uv=v.iter().zip(weights).fold(egui::Vec2::ZERO,|sum,(v,w)|sum+v.uv.to_vec2()*w);
            let sampled=sample(texture,uv);
            let mut tint=[0.0;4];
            for (vertex,w) in v.iter().zip(weights) { for (channel,value) in tint.iter_mut().zip(vertex.color.to_array()) { *channel+=w*f32::from(value)/255.0; } }
            let alpha=(sampled[3]*tint[3]).clamp(0.0,1.0);
            let target=image.get_pixel_mut(x,y);
            for channel in 0..3 { target[channel]=((sampled[channel]*tint[channel]+f32::from(target[channel])/255.0*(1.0-alpha))*255.0).round().clamp(0.0,255.0) as u8; }
            target[3]=255;
        }
    }
}

fn sample(texture:&ColorImage,uv:egui::Vec2)->[f32;4] {
    if texture.size[0]==0 || texture.size[1]==0 || !uv.x.is_finite() || !uv.y.is_finite() {return [0.0;4];}
    let x=(uv.x*texture.size[0] as f32-0.5).clamp(0.0,(texture.size[0]-1) as f32);
    let y=(uv.y*texture.size[1] as f32-0.5).clamp(0.0,(texture.size[1]-1) as f32);
    let x0=x.floor() as usize; let y0=y.floor() as usize;
    let x1=(x0+1).min(texture.size[0]-1);let y1=(y0+1).min(texture.size[1]-1);
    let dx=x-x0 as f32;let dy=y-y0 as f32;let mut out=[0.0;4];
    for ((x,y),weight) in [((x0,y0),(1.0-dx)*(1.0-dy)),((x1,y0),dx*(1.0-dy)),((x0,y1),(1.0-dx)*dy),((x1,y1),dx*dy)] {
        if let Some(pixel)=texture.pixels.get(y*texture.size[0]+x) { for (target,value) in out.iter_mut().zip(pixel.to_array()) { *target+=weight*f32::from(value)/255.0; } }
    }
    out
}
