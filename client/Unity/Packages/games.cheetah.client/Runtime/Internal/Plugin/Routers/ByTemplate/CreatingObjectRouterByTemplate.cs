using Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate.Abstract;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate
{
    /// <summary>
    /// Маршрутизация событий создания объекта с фильтрацией по шаблону
    /// </summary>
    public class CreatingObjectRouterByTemplate : AbstractObjectEventRouterByTemplate
    {
        private ObjectCommandRouter objectCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectCreatingListener += OnObjectCreating;
        }

        private void OnObjectCreating(in CheetahObjectId objectId, ushort template)
        {
            Notify(in objectId);
        }
    }
}