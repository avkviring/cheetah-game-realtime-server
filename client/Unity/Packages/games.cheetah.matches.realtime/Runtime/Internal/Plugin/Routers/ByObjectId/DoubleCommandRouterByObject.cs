using Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.ByObjectId
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

        private void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, double value)
        {
            Notify(commandCreator, in objectId, fieldId, ref value);
        }
    }
}