using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal.Plugin.Routers.ByObjectId;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.DOA.Income.ByObject
{
    public class StructureIncomeByObjectCommandCollector<T> : AbstractIncomeByObjectCommandCollector<T>
    {
        private CheetahObjectId id;
        private readonly StructureCommandRouterByObject router;
        private readonly Codec<T> codec;

        public StructureIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, FieldId.Structure fieldId) : base(client, fieldId.Id)
        {
            this.id = id;
            router = client.GetPlugin<StructureCommandRouterByObject>();
            router.RegisterListener(in id, fieldId.Id, OnStructure);
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
            router.UnRegisterListener(in id, fieldId, OnStructure);
        }
    }
}