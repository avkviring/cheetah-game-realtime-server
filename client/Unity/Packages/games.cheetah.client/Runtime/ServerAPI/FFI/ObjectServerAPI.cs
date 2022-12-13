using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class ObjectServerAPI : IObjectServerAPI
    {

        public byte SetCreateListener(ushort clientId, IObjectServerAPI.CreateListener listener)
        {
            return ObjectFFI.SetCreateListener(clientId, listener);
        }

        public byte SetCreatedListener(ushort clientId, IObjectServerAPI.CreatedListener listener)
        {
            return ObjectFFI.SetCreatedListener(clientId, listener);
        }

        public byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref CheetahObjectId objectId)
        {
            return ObjectFFI.CreateObject(clientId, template, accessGroup, ref objectId);
        }

        public byte CreatedObject(ushort clientId, in CheetahObjectId objectId, bool roomOwner, ref CheetahBuffer singletonKey)
        {
            return ObjectFFI.Created(clientId, in objectId, roomOwner, ref singletonKey);
        }

        public byte SetDeleteListener(ushort clientId, IObjectServerAPI.DeleteListener objectDeleteListener)
        {
            return ObjectFFI.SetDeleteListener(clientId, objectDeleteListener);
        }

        public byte Delete(ushort clientId, in CheetahObjectId objectId)
        {
            return ObjectFFI.Delete(clientId, in objectId);
        }
    }
}