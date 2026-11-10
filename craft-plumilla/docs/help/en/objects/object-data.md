# Object data

The [[docker.object_data]] panel attaches your own information to an object,
as named fields: a part number, a price, a note. The data is saved with the
drawing and goes with the object when you copy it.

![The Object Data panel](../../img/objects-object-data.png)

## Adding and editing data {#fields}

1. Select an object. With nothing selected, the panel says
   [[docker.select_one_object]].
2. The panel lists the object's fields in two columns, [[docker.field]] and
   [[docker.value]]. Click a name or a value to edit it.
3. Click [[docker.add_field]] to add a field, or one of the buttons for a
   common field: Name, Cost, Comments and CDRStaticID.
4. Click the **x** at the end of a row to remove the field.

When the selected objects have a Cost field with a number, the panel shows
their total cost.

Some features keep their own data in objects, such as dimension lines,
callouts, connectors and anchors; you rarely need to change those fields by
hand.
