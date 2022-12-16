using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Internal.Plugin.Routers.ByObjectId;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.DOA.Income.ByObject
{
    public class EventIncomeByObjectCommandCollector<T> : AbstractIncomeByObjectCommandCollector<T>
    {
        private CheetahObjectId id;
        private readonly EventCommandRouterByObject router;
        private readonly Codec<T> codec;

        public EventIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, FieldId.Event fieldId) : base(client, fieldId.Id)
        {
            this.id = id;
            router = client.GetPlugin<EventCommandRouterByObject>();
            router.RegisterListener(in id, fieldId.Id, OnEvent);
            codec = client.CodecRegistry.GetCodec<T>();
        }


        private void OnEvent(ushort commandCreator, CheetahObject cheetahObject, bool created, ref CheetahBuffer data)
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
            router.UnRegisterListener(in id, fieldId, OnEvent);
        }
    }
}