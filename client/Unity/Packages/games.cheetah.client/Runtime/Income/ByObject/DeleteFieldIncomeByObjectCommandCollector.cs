using Games.Cheetah.Client.Internal.Plugin.Routers.ByObjectId;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.DOA.Income.ByObject
{
    public class DeleteFieldIncomeByObjectCommandCollector : AbstractIncomeByObjectCommandCollector<FieldType>
    {
        private CheetahObjectId id;
        private readonly DeleteFieldCommandRouterByObjec router;

        public DeleteFieldIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, ushort fieldId) : base(client, fieldId)
        {
            this.id = id;
            router = client.GetPlugin<DeleteFieldCommandRouterByObjec>();
            router.RegisterListener(in id, fieldId, OnChange);
        }


        private void OnChange(ushort commandCreator, CheetahObject cheetahObject, bool created, ref FieldType fieldType)
        {
            if (!created) return;
            ref var newItem = ref stream.Add();
            newItem.commandCreator = commandCreator;
            newItem.value = fieldType;
        }

        public override void Dispose()
        {
            base.Dispose();
            router.UnRegisterListener(in id, fieldId, OnChange);
        }
    }
}