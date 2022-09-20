using System;
using AOT;
using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI
{
    /// <summary>
    /// Маршрутизация событий жизненного цикла игрового объекта из RelayClient-а произвольным подписчикам
    /// </summary>
    public class ObjectCommandRouter : Plugin
    {
        private static ObjectCommandRouter current;
        private CheetahClient client;
        internal event ObjectFFI.CreateListener ObjectCreatingListener;
        internal event ObjectFFI.CreatedListener ObjectCreatedListener;
        internal event ObjectFFI.DeleteListener ObjectDeleteListener;
        internal event ObjectFFI.DeleteListener ObjectPostDeleteListener;


        public void Init(CheetahClient client)
        {
            client.BeforeUpdateHook += BeforeUpdate;
            this.client = client;
            ObjectFFI.SetCreateListener(client.Id, OnCreateListener);
            ObjectFFI.SetCreatedListener(client.Id, OnCreatedListener);
            ObjectFFI.SetDeleteListener(client.Id, OnDeleteListener);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(ObjectFFI.CreateListener))]
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

        [MonoPInvokeCallback(typeof(ObjectFFI.CreatedListener))]
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


        [MonoPInvokeCallback(typeof(ObjectFFI.DeleteListener))]
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