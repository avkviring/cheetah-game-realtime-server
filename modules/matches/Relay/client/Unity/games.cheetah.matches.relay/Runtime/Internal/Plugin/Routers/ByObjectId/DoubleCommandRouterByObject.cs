using Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.ByObjectId
{
    public class DoubleCommandRouterByObject : AbstractRouterByObject<double>
    {
        private DoubleCommandRouter doubleCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            doubleCommandRouter = client.GetPlugin<DoubleCommandRouter>();
            doubleCommandRouter.ChangeListener += OnChange;
        }

        private void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, double value)
        {
            Notify(commandCreator, ref objectId, fieldId, ref value);
        }
    }
}