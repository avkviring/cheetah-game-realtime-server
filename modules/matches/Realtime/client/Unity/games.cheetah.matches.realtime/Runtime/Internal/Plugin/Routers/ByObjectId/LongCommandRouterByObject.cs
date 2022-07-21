using Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.ByObjectId
{
    public class LongCommandRouterByObject : AbstractRouterByObject<long>
    {
        private LongCommandRouter longCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            longCommandRouter = client.GetPlugin<LongCommandRouter>();
            longCommandRouter.ChangeListener += OnChange;
        }

        private void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, long value)
        {
            Notify(commandCreator, ref objectId, fieldId, ref value);
        }
    }
}