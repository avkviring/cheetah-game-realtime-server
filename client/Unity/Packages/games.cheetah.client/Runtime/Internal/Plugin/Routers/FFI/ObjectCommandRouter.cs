using System;
using AOT;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    /// <summary>
    /// Маршрутизация событий жизненного цикла игрового объекта из RelayClient-а произвольным подписчикам
    /// </summary>
    public class ObjectCommandRouter : Plugin
    {
        private static ObjectCommandRouter current;
        private CheetahClient client;
        internal event IObjectServerAPI.CreateListener ObjectCreatingListener;
        internal event IObjectServerAPI.CreatedListener ObjectCreatedListener;
        internal event IObjectServerAPI.DeleteListener ObjectDeleteListener;
        internal event IObjectServerAPI.DeleteListener ObjectPostDeleteListener;


        public void Init(CheetahClient client)
        {
            client.BeforeUpdateHook += BeforeUpdate;
            this.client = client;
            client.serverAPI.Object.SetCreateListener(client.Id, OnCreateListener);
            client.serverAPI.Object.SetCreatedListener(client.Id, OnCreatedListener);
            client.serverAPI.Object.SetDeleteListener(client.Id, OnDeleteListener);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(IObjectServerAPI.CreateListener))]
        private static void OnCreateListener(in CheetahObjectId objectId, ushort template)
        {
            try
            {
                current.ObjectCreatingListener?.Invoke(in objectId, template);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }

        [MonoPInvokeCallback(typeof(IObjectServerAPI.CreatedListener))]
        private static void OnCreatedListener(in CheetahObjectId objectId)
        {
            try
            {
                current.ObjectCreatedListener?.Invoke(in objectId);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }


        [MonoPInvokeCallback(typeof(IObjectServerAPI.DeleteListener))]
        private static void OnDeleteListener(in CheetahObjectId objectId)
        {
            try
            {
                current.ObjectDeleteListener?.Invoke(in objectId);
                current.ObjectPostDeleteListener?.Invoke(in objectId);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}