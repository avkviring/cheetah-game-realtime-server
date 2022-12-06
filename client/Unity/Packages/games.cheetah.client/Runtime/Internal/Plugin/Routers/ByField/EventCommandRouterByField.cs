using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByField
{
    public class EventRouterByField : AbstractRouterByField<CheetahBuffer>
    {
        private EventCommandRouter eventCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            eventCommandRouter = client.GetPlugin<EventCommandRouter>();
            eventCommandRouter.NewEventListener += OnNewEvent;
        }


        private void OnNewEvent(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            Notify(commandCreator, in objectId, fieldId, ref data);
        }
    }
}