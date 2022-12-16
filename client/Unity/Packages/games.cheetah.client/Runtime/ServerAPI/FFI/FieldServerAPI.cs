using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class FieldServerAPI : IFieldServerAPI
    {
        public byte SetListener(ushort clientId, IFieldServerAPI.Listener listener)
        {
            return FieldFFI.SetListener(clientId, listener);
        }

        public byte Delete(ushort clientId, in CheetahObjectId objectId, FieldId fieldId)
        {
            return FieldFFI.Delete(clientId, in objectId, fieldId.Id, fieldId.Type);
        }
    }
}