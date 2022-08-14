using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByField;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.DOA.Income.ByField
{
    /// <summary>
    /// Поток изменений структур c сервера, сбрасывается каждый кадр
    /// </summary>
    public class StructureIncomeByFieldCommandCollector<T> : AbstractIncomeByFieldCommandCollector<T>
    {
        private readonly StructCommandRouterByField router;
        private readonly Codec<T> codec;

        public StructureIncomeByFieldCommandCollector(CheetahClient client, ushort fieldId) : base(client, fieldId)
        {
            router = client.GetPlugin<StructCommandRouterByField>();
            router.RegisterListener(fieldId, OnStructure);
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