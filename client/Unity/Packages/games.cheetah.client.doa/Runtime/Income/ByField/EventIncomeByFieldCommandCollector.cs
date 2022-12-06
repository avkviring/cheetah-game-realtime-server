using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal.Plugin.Routers.ByField;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.DOA.Income.ByField
{
    /// <summary>
    /// Поток событий с сервера, сбрасывается каждый кадр
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public class EventIncomeByFieldCommandCollector<T> : AbstractIncomeByFieldCommandCollector<T>
    {
        private readonly EventRouterByField router;
        private readonly Codec<T> codec;


        public EventIncomeByFieldCommandCollector(CheetahClient client, ushort fieldId) : base(client, fieldId)
        {
            router = client.GetPlugin<EventRouterByField>();
            router.RegisterListener(fieldId, OnEvent);
            codec = client.CodecRegistry.GetCodec<T>();
        }


        private void OnEvent(ushort commandCreator, CheetahObject cheetahObject, bool created, ref CheetahBuffer data)
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
            router.UnRegisterListener(fieldId, OnEvent);
            base.Dispose();
        }
    }
}