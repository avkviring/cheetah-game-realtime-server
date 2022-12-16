using Games.Cheetah.Client.Internal.Plugin.Routers.ByField;

namespace Games.Cheetah.Client.DOA.Income.ByField
{
    /// <summary>
    /// Поток изменений структур double, сбрасывается каждый кадр
    /// </summary>
    public class DoubleIncomeByFieldCommandCollector : AbstractIncomeByFieldCommandCollector<double>
    {
        private readonly DoubleCommandRouterByField router;

        public DoubleIncomeByFieldCommandCollector(CheetahClient client, FieldId.Double fieldId) : base(client, fieldId.Id)
        {
            router = client.GetPlugin<DoubleCommandRouterByField>();
            router.RegisterListener(fieldId.Id, OnStructure);
        }


        private void OnStructure(ushort commandCreator, CheetahObject cheetahObject, bool created, ref double data)
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
            router.UnRegisterListener(fieldId, OnStructure);
        }
    }
}