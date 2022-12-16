using Games.Cheetah.Client.Internal.Plugin.Routers.ByField;

namespace Games.Cheetah.Client.DOA.Income.ByField
{
    /// <summary>
    /// Поток изменений структур long, сбрасывается каждый кадр
    /// </summary>
    public class LongIncomeByFieldCommandCollector : AbstractIncomeByFieldCommandCollector<long>
    {
        private readonly LongCommandRouterByField router;

        public LongIncomeByFieldCommandCollector(CheetahClient client, FieldId.Long fieldId) : base(client, fieldId.Id)
        {
            router = client.GetPlugin<LongCommandRouterByField>();
            router.RegisterListener(fieldId.Id, OnChange);
        }


        private void OnChange(ushort commandCreator, CheetahObject cheetahObject, bool created, ref long data)
        {
            if (!created) return;
            ref var newItem = ref stream.Add();
            newItem.commandCreator = commandCreator;
            newItem.cheetahObject = cheetahObject;
            newItem.value = data;
        }

        public override void Dispose()
        {
            base.Dispose();
            router.UnRegisterListener(fieldId, OnChange);
        }
    }
}