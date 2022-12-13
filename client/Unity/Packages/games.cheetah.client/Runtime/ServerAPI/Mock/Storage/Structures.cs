using Games.Cheetah.Client.ServerAPI.Mock.Type;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public class Structures : AbstractStorage<CheetahBuffer>, IStructureServerAPI
    {
        internal IStructureServerAPI.Listener listener;

        public byte SetListener(ushort clientId, IStructureServerAPI.Listener listener)
        {
            this.listener = listener;
            return 0;
        }

        public byte CompareAndSet(
            ushort clientId,
            in CheetahObjectId objectId,
            ushort fieldId,
            ref CheetahBuffer currentValue,
            ref CheetahBuffer newValue,
            bool hasReset,
            ref CheetahBuffer resetValue)
        {
            var key = new FieldKey(objectId, fieldId);
            if (fields.TryGetValue(key, out var data) && data.Equals(currentValue))
            {
                fields[key] = newValue;
            }

            return 0;
        }
    }
}