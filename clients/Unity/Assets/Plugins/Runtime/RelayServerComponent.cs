using System;
using Cheetach.Relay;
using UnityEngine;

namespace Runtime
{
    public class RelayServerComponent : MonoBehaviour
    {
        private ushort client = 0;

        private void Start()
        {
            client = LowLevelApi.CreateClient("127.0.0.1:5000", "room", "client");
        }


        private void Update()
        {
           // LowLevelApi.GetConnectionStatus(client, this.OnNetworkStatus, this.OnError);
        }

        private void OnNetworkStatus(NetworkStatus networkStatus)
        {
            Debug.Log("Network status " + networkStatus);
        }

        private void OnError()
        {
            Debug.LogError("OnError");
        }

        private void OnDestroy()
        {
            Debug.Log("Destroy client");
            LowLevelApi.DestroyClient(client);
        }
    }
}