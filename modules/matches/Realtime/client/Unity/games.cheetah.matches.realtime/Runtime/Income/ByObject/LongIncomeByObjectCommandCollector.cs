using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Income.ByObject
{
    public class LongIncomeByObjectCommandCollector : AbstractIncomeByObjectCommandCollector<long>
    {
        private CheetahObjectId id;
        private readonly LongCommandRouterByObject router;

        public LongIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, ushort fieldId) : base(client, fieldId)
        {
            this.id = id;
            router = client.GetPlugin<LongCommandRouterByObject>();
            router.RegisterListener(ref id, fieldId, OnChange);
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
            router.UnRegisterListener(ref id, fieldId, OnChange);
        }
    }
}