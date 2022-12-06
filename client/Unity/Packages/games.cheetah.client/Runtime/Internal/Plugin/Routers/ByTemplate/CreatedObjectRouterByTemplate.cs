using Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate.Abstract;
using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByTemplate
{
    /// <summary>
    /// Маршрутизация событий окончания создания объекта с фильтрацией по шаблону
    /// </summary>
    public class CreatedObjectRouterByTemplate : AbstractObjectEventRouterByTemplate
    {
        private ObjectCommandRouter objectCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            objectCommandRouter = client.GetPlugin<ObjectCommandRouter>();
            objectCommandRouter.ObjectCreatedListener += OnObjectCreated;
        }

        private void OnObjectCreated(in CheetahObjectId objectId)
        {
            Notify(in objectId);
        }
    }
}