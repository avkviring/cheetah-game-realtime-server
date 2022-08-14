using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.DOA.Income.ByObject
{
    public class DeleteFieldIncomeByObjectCommandCollector : AbstractIncomeByObjectCommandCollector<FieldType>
    {
        private CheetahObjectId id;
        private readonly DeleteFieldCommandRouterByObjec router;

        public DeleteFieldIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, ushort fieldId) : base(client, fieldId)
        {
            this.id = id;
            router = client.GetPlugin<DeleteFieldCommandRouterByObjec>();
            router.RegisterListener(ref id, fieldId, OnChange);
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
            router.UnRegisterListener(ref id, fieldId, OnChange);
        }
    }
}