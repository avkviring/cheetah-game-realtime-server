namespace CheetahRelay
{
    public class CheetahObjectBuilder
    {
        public ulong accessGroup;
        public ushort template;
        public GameObjectFields fields;


        public CheetahObjectBuilder SetAccessGroup(ulong accessGroup)
        {
            this.accessGroup = accessGroup;
            return this;
        }

        public CheetahObjectBuilder SetStructure(ushort fieldId, ref CheetahBuffer data)
        {
            unsafe
            {
                var index = fields.structures.count;
                fields.structures.fields[index] = fieldId;
                fields.structures.sizes[index] = data.size;

                // TODO - оптимизировать
                var offset = index * Const.MaxSizeStruct;
                for (var i = 0; i < data.size; i++)
                {
                    fields.structures.values[offset + i] = data.values[i];
                }

                fields.structures.count++;
            }

            return this;
        }

        public CheetahObjectBuilder SetLong(ushort fieldId, long value)
        {
            unsafe
            {
                var index = fields.longs.count;
                fields.longs.fields[index] = fieldId;
                fields.longs.values[index] = value;
                fields.longs.count++;
            }

            return this;
        }

        public CheetahObjectBuilder SetDouble(ushort fieldId, double value)
        {
            unsafe
            {
                var index = fields.doubles.count;
                fields.doubles.fields[index] = fieldId;
                fields.doubles.values[index] = value;
                fields.doubles.count++;
            }

            return this;
        }

        public CheetahObjectBuilder SetTemplate(ushort template)
        {
            this.template = template;
            return this;
        }

        public CheetahObjectId? BuildAndSendToServer()
        {
            CheetahObject.Create(template, accessGroup, ref fields, out var objectId);
            return objectId;
        }
    }
}