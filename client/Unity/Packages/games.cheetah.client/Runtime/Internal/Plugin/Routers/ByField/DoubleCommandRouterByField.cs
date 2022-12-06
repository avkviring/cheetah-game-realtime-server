using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByField
{
    public class DoubleCommandRouterByField : AbstractRouterByField<double>
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