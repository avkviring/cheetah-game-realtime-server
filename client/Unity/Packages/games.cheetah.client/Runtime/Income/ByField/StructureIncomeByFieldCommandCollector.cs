using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal.Plugin.Routers.ByField;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.DOA.Income.ByField
{
    /// <summary>
    /// Поток изменений структур c сервера, сбрасывается каждый кадр
    /// </summary>
    public class StructureIncomeByFieldCommandCollector<T> : AbstractIncomeByFieldCommandCollector<T>
    {
        private readonly StructCommandRouterByField router;
        private readonly Codec<T> codec;

        public StructureIncomeByFieldCommandCollector(CheetahClient client, FieldId.Structure fieldId) : base(client, fieldId.Id)
        {
            router = client.GetPlugin<StructCommandRouterByField>();
            router.RegisterListener(fieldId.Id, OnStructure);
            codec = client.CodecRegistry.GetCodec<T>();
        }


        private void OnStructure(ushort commandCreator, CheetahObject cheetahObject, bool created, ref CheetahBuffer data)
        {
            if (!created) return;
            ref var newItem = ref stream.Add();
            newItem.commandCreator = commandCreator;
            newItem.cheetahObject = cheetahObject;
            data.pos = 0;
            codec.Decode(ref data, ref newItem.value);
        }

        public override void Dispose()
        {
            base.Dispose();
            router.UnRegisterListener(fieldId, OnStructure);
        }
    }
}