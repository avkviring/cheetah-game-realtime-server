using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class CreateObjectCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;
        private readonly ushort template;
        private ulong accessGroup;

        public CreateObjectCommandFromServer(CheetahObjectId objectId, ushort template, ulong accessGroup = default)
        {
            this.objectId = objectId;
            this.template = template;
            this.accessGroup = accessGroup;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            var objectStorage = cheetahClientMock.objects;
            var objectIdCopy = objectId;
            objectStorage.CreateObject(0, template, accessGroup, ref objectIdCopy);
            objectStorage.createListener?.Invoke(in objectId, template);
        }
    }
}