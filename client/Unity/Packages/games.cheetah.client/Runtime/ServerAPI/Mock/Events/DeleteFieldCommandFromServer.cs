using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class DeleteFieldCommandFromServer : ICommandFromServer
    {
        private readonly CheetahObjectId objectId;
        private readonly FieldType fieldType;
        private readonly ushort fieldId;


        public DeleteFieldCommandFromServer(CheetahObjectId objectId, FieldType fieldType, ushort fieldId)
        {
            this.objectId = objectId;
            this.fieldType = fieldType;
            this.fieldId = fieldId;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            var fieldStorage = cheetahClientMock.fields;
            fieldStorage.Delete(0, in objectId, fieldId, fieldType);
            fieldStorage.listener?.Invoke(0, in objectId, fieldId, fieldType);
        }
    }
}