using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class StructureServerAPI : IStructureServerAPI
    {
        public byte SetListener(ushort clientId, IStructureServerAPI.Listener listener)
        {
            return StructureFFI.SetListener(clientId, listener);
        }

        public byte Set(ushort clientId, in CheetahObjectId objectId, FieldId.Structure fieldId, ref CheetahBuffer data)
        {
            return StructureFFI.Set(clientId, in objectId, fieldId.Id, ref data);
        }

        public byte CompareAndSet(
            ushort clientId,
            in CheetahObjectId objectId,
            FieldId.Structure fieldId,
            ref CheetahBuffer currentValue,
            ref CheetahBuffer newValue,
            bool hasReset,
            ref CheetahBuffer resetValue
        )
        {
            return StructureFFI.CompareAndSet(clientId, in objectId, fieldId.Id, ref currentValue, ref newValue, hasReset, ref resetValue);
        }
    }
}