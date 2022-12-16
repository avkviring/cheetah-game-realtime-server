using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Storage
{
    public class Events : AbstractStorage<CheetahBuffer, FieldId.Event>, IEventServerAPI
    {
        private IEventServerAPI.Listener listener;

        public byte SetListener(ushort clientId, IEventServerAPI.Listener listener)
        {
            this.listener = listener;
            return 0;
        }

        public byte Send(ushort clientId, in CheetahObjectId objectId, FieldId.Event fieldId, ref CheetahBuffer data)
        {
            return Set(clientId, in objectId, fieldId, ref data);
        }

        public byte Send(ushort clientId, ushort targetUser, in CheetahObjectId objectId, FieldId.Event fieldId, ref CheetahBuffer data)
        {
            return Set(clientId, in objectId, fieldId, ref data);
        }
    }
}