# Bevels

A bevel makes the edges of an object look raised or carved, by lighting one
side of the edge and shading the other. It is a live effect, set in the
[[docker.bevel]] panel; open it with [[menu.effects > menu.effects.bevel]].

![The Bevel panel](../../img/effects-bevel.png)

## To bevel an object {#create}

1. Select a filled object.
2. In the [[docker.bevel]] panel, choose the style:
   - [[docker.soft_edge]]: the edge slopes into the object, lit on one side
     and shaded on the other;
   - [[docker.emboss]]: the object seems pressed out of the page; the
     bevel is drawn around the outline instead of inside it.
3. Set the look of the bevel:
   - [[docker.depth]]: how far the bevel reaches from the edge, into the
     object for a soft edge and out of it for an emboss;
   - [[docker.level]]: how many steps the light and the shade are cut into
     across the bevel. 1 gives two flat tones, more fade them smoothly from
     the edge inwards;
   - [[docker.light_angle]]: the direction the light comes from;
   - [[docker.intensity]]: how strong the light and the shade are;
   - [[docker.light_color]] and [[docker.shadow_color]]: the colours of the
     lit and the shaded side;
   - [[docker.emboss_color]]: the colour the bevel is toned from. Tick the
     box and pick a colour to use your own; left off, the object's own
     fill is used.
4. Click [[docker.apply]].

When you select an object that already has a bevel, the panel shows that
bevel, and changing a setting edits it right away.

To copy a bevel from another object, use
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] removes it. The extrusion panel
also has a bevel for the front face; see [Extrusions](extrude.md#settings).
