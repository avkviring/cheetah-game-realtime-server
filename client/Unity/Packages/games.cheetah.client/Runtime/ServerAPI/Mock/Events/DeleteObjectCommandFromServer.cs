using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class DeleteObjectCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;

        public DeleteObjectCommandFromServer(CheetahObjectId objectId)
        {
            this.objectId = objectId;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            var objectStorage = cheetahClientMock.objects;
            objectStorage.deleteListener?.Invoke(in objectId);
            objectStorage.Delete(0, in objectId);
        }
    }
}