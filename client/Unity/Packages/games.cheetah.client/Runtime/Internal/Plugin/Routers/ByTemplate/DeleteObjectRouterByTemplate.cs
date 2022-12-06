using Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate.Abstract;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate
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

        private void OnObjectDelete(in CheetahObjectId objectId)
        {
            Notify(in objectId);
        }
    }
}