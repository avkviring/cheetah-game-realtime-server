using Games.Cheetah.Client.Internal.Plugin.Routers.ByObjectId;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.DOA.Income.ByObject
{
    public class LongIncomeByObjectCommandCollector : AbstractIncomeByObjectCommandCollector<long>
    {
        private CheetahObjectId id;
        private readonly LongCommandRouterByObject router;

        public LongIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, FieldId.Long fieldId) : base(client, fieldId.Id)
        {
            this.id = id;
            router = client.GetPlugin<LongCommandRouterByObject>();
            router.RegisterListener(in id, fieldId.Id, OnChange);
        }


        private void OnChange(ushort commandCreator, CheetahObject cheetahObject, bool created, ref long data)
        {
            if (!created) return;
            ref var newItem = ref stream.Add();
            newItem.commandCreator = commandCreator;
            newItem.value = data;
        }

        public override void Dispose()
        {
            base.Dispose();
            router.UnRegisterListener(in id, fieldId, OnChange);
        }
    }
}