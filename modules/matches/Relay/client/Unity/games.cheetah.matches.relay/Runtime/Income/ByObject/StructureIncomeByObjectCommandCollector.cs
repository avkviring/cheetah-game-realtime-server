using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Internal.Plugin.Routers.ByObjectId;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Income.ByObject
{
    public class StructureIncomeByObjectCommandCollector<T> : AbstractIncomeByObjectCommandCollector<T>
    {
        private CheetahObjectId id;
        private readonly StructureCommandRouterByObject router;
        private readonly Codec<T> codec;

        public StructureIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, ushort fieldId) : base(client, fieldId)
        {
            this.id = id;
            router = client.GetPlugin<StructureCommandRouterByObject>();
            router.RegisterListener(ref id, fieldId, OnStructure);
            codec = client.CodecRegistry.GetCodec<T>();
        }


        private void OnStructure(ushort commandCreator, CheetahObject cheetahObject, bool created, ref CheetahBuffer data)
        {
            if (!created) return;
            ref var newItem = ref stream.Add();
            newItem.commandCreator = commandCreator;
            data.pos = 0;
            codec.Decode(ref data, ref newItem.value);
        }

        public override void Dispose()
        {
            base.Dispose();
            router.UnRegisterListener(ref id, fieldId, OnStructure);
        }
    }
}