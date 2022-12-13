using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class CreatedObjectCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;
        private readonly bool roomOwner;
        private CheetahBuffer singletonKey;

        public CreatedObjectCommandFromServer(CheetahObjectId objectId, bool roomOwner = false, CheetahBuffer singletonKey = default)
        {
            this.objectId = objectId;
            this.roomOwner = roomOwner;
            this.singletonKey = singletonKey;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            var objectStorage = cheetahClientMock.objects;
            objectStorage.CreatedObject(0, in objectId, roomOwner, ref singletonKey);
            objectStorage.createdListener?.Invoke(in objectId);
        }
    }
}