# Joining curves

The [[docker.join_curves]] panel closes the gaps between the ends of curves,
for example lines that should meet at a corner but stop short of it. Open it
with [[menu.object > menu.object.join_curves]].

![The Join Curves panel](../../img/lines-shapes-join-curves.png)

## To join curves {#join}

1. Select the curves. Open parts inside one curve are joined too.
2. Choose how the ends are joined:
   - **Extend**: both ends carry on in their directions until they meet, as a
     corner. When the directions never meet ahead of both ends, a straight
     line joins them.
   - **Chamfer**: a straight line between the ends.
   - **Fillet**: as Extend, then the corner is rounded with the **radius**
     (2 mm at first).
   - **Bezier curve**: a smooth curve that leaves and arrives in the
     directions of the ends.
3. Set the **gap tolerance** (5 mm at first). Ends further apart are not
   joined.
4. Click [[docker.apply]].

The closest pair of ends is joined first, then the next, until no pair is left
within the tolerance; the two ends of one part close it. The joined curve
takes the place, fill and outline of the curve you selected last, and the
other curves are removed. When no ends lie close enough, nothing changes and
the status bar says so.

You can also join two end nodes of one curve with the [[tool.shape]] tool;
see [Shaping curves with the Shape tool](shape-tool.md#property-bar). Drawing
from the end of a selected curve also continues it; see
[Continuing a curve](lines.md#continue).
