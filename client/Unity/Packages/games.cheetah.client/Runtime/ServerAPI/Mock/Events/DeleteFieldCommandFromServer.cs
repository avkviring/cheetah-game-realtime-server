using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class DeleteFieldCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;
        private readonly FieldId fieldId;


        public DeleteFieldCommandFromServer(CheetahObjectId objectId, FieldId fieldId)
        {
            this.objectId = objectId;
            this.fieldId = fieldId;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            var fieldStorage = cheetahClientMock.fields;
            fieldStorage.Delete(0, in objectId, fieldId);
            fieldStorage.listener?.Invoke(0, in objectId, fieldId.Id, fieldId.Type);
        }
    }
}