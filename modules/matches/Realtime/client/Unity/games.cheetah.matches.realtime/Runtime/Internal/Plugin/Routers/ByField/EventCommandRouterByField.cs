using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByField
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


        private void OnNewEvent(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            Notify(commandCreator, ref objectId, fieldId, ref data);
        }
    }
}