using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId
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

        private void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer value)
        {
            Notify(commandCreator, in objectId, fieldId, ref value);
        }
    }
}