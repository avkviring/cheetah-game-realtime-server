using Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.ByObjectId
{
    public class EventCommandRouterByObject : AbstractRouterByObject<CheetahBuffer>
    {
        private EventCommandRouter eventCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            eventCommandRouter = client.GetPlugin<EventCommandRouter>();
            eventCommandRouter.NewEventListener += OnChange;
        }

        private void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer value)
        {
            Notify(commandCreator, ref objectId, fieldId, ref value);
        }
    }
}