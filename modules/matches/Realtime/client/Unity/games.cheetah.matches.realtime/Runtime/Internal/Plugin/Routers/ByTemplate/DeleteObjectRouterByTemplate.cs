using Cheetah.Matches.Relay.Internal.Plugin.Routers.ByTemplate.Abstract;
using Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.ByTemplate
{
    public class DeleteObjectRouterByTemplate : AbstractObjectEventRouterByTemplate
    {
        private ObjectCommandRouter objectCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectDeleteListener += OnObjectDelete;
        }

        private void OnObjectDelete(ref CheetahObjectId objectId)
        {
            Notify(ref objectId);
        }
    }
}