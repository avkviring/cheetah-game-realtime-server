using System.Collections.Generic;
using Games.Cheetah.Client.ServerAPI.Mock.Type;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public class Doubles : AbstractStorage<double, FieldId.Double>, IDoubleServerAPI
    {
        internal IDoubleServerAPI.Listener listener;

        public byte SetListener(ushort clientId, IDoubleServerAPI.Listener listener)
        {
            this.listener = listener;
            return 0;
        }

        public byte Set(ushort clientId, in CheetahObjectId objectId, FieldId.Double fieldId, double value)
        {
            return Set(clientId, in objectId, fieldId, ref value);
        }


        public byte Increment(ushort clientId, in CheetahObjectId objectId, FieldId.Double fieldId, double increment)
        {
            var fieldKey = new FieldKey<FieldId.Double>(objectId, fieldId);
            fields[fieldKey] = fields.GetValueOrDefault(fieldKey) + increment;
            return 0;
        }
    }
}