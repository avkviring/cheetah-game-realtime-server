using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.DOA.Income.ByObject
{
    public class EventIncomeByObjectCommandCollector<T> : AbstractIncomeByObjectCommandCollector<T>
    {
        private CheetahObjectId id;
        private readonly EventCommandRouterByObject router;
        private readonly Codec<T> codec;

        public EventIncomeByObjectCommandCollector(CheetahClient client, CheetahObjectId id, ushort fieldId) : base(client, fieldId)
        {
            this.id = id;
            router = client.GetPlugin<EventCommandRouterByObject>();
            router.RegisterListener(ref id, fieldId, OnEvent);
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
            router.UnRegisterListener(ref id, fieldId, OnEvent);
        }
    }
}