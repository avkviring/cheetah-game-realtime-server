using Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.ByObjectId
{
    public class StructureCommandRouterByObject : AbstractRouterByObject<CheetahBuffer>
    {
        private StructCommandRouter structCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            structCommandRouter = client.GetPlugin<StructCommandRouter>();
            structCommandRouter.ChangeListener += OnChange;
        }

        private void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer value)
        {
            Notify(commandCreator, ref objectId, fieldId, ref value);
        }
    }
}