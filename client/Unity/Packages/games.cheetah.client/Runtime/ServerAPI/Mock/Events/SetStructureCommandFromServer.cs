using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.Mock.Events
{
    public class SetStructureCommandFromServer<T> : ICommandFromServer where T : unmanaged
    {
        private readonly CheetahObjectId objectId;
        private readonly FieldId.Structure fieldId;
        private readonly T value;

        public SetStructureCommandFromServer(CheetahObjectId objectId,  FieldId.Structure fieldId, T value)
        {
            this.objectId = objectId;
            this.fieldId = fieldId;
            this.value = value;
        }

        public void Apply(CheetahClientMock cheetahClientMock)
        {
            var buffer = new CheetahBuffer();
            cheetahClientMock.codecRegistry.GetCodec<T>().Encode(in value, ref buffer);
            cheetahClientMock.structures.listener?.Invoke(0, in objectId, fieldId.Id, ref buffer);
        }
    }
}