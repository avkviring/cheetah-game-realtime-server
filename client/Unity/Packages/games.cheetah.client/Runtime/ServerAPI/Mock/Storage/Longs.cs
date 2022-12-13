using System.Collections.Generic;
using Games.Cheetah.Client.ServerAPI.Mock.Type;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public class Longs : AbstractStorage<long>, ILongServerAPI
    {
        internal ILongServerAPI.Listener listener;

        public byte SetListener(ushort clientId, ILongServerAPI.Listener listener)
        {
            this.listener = listener;
            return 0;
        }

        public byte Set(ushort clientId, in CheetahObjectId objectId, ushort fieldId, long value)
        {
            return Set(clientId, in objectId, fieldId, ref value);
        }

        public byte Increment(ushort clientId, in CheetahObjectId objectId, ushort fieldId, long increment)
        {
            var fieldKey = new FieldKey(objectId, fieldId);
            fields[fieldKey] = fields.GetValueOrDefault(fieldKey) + increment;
            return 0;
        }

        public byte CompareAndSet(ushort clientId, in CheetahObjectId objectId, ushort fieldId, long currentValue, long newValue, bool hasReset,
            long resetValue)
        {
            var fieldKey = new FieldKey(objectId, fieldId);
            if (fields.TryGetValue(fieldKey, out var value) && value == currentValue)
            {
                fields[fieldKey] = newValue;
            }

            return 0;
        }
    }
}