using System;
using AOT;
using CheetahRelay.Runtime.Client.Codec;
using MessagePack;
using UnityEngine;
using UnityEngine.UI;

namespace CheetahRelay.Application
{
    /**
 * Тестирование основных возможностей RelayClient в сборке клиента
 */
    [MessagePackObject]
    public class EventWithMessage
    {
        [Key(0)] public string Message { get; set; }
    }

    [MessagePackObject]
    public class StructureWithMessage
    {
        [Key(0)] public string Message { get; set; }
    }


    public class RelayTestComponent : MonoBehaviour
    {
        [SerializeField] public Text objectField;
        [SerializeField] public Text eventField;
        [SerializeField] public Text longField;

        private CodecRegistry codecRegistry = new CodecRegistry();
        
        protected ushort ClientA;
        protected ushort ClientB;
        protected CheetahObjectId ObjectId;
        protected static RelayTestComponent instance;

        private void Start()
        {
            instance = this;
            
            Debug.Log("Cheetah Start");
            LoggerGateway.Init();
            LoggerExternals.SetMaxLogLevel(CheetahLogLevel.Warn);

            var privateKey = CreatePrivateKey();
            Debug.Log("Cheetah created private key");
            CheetahClient.CreateClient("192.168.212.97:5000", 1, ref privateKey, 0, out ClientA);
            CheetahClient.CreateClient("192.168.212.97:5000", 2, ref privateKey, 0, out ClientB);
            Debug.Log("Cheetah create clients");

            CheetahClient.SetCurrentClient(ClientA);
            CheetahObject.Create(55, 1, ref ObjectId);
            CheetahObject.Created(ref ObjectId);
            Debug.Log("Cheetah created objects");
            
            CheetahClient.SetCurrentClient(ClientB);
            CheetahObject.SetCreatedListener(OnCreate);
            CheetahClient.AttachToRoom();
            Debug.Log("Cheetah attached to room");
            
        }

        private static CheetahBuffer CreatePrivateKey()
        {
            var privateKey = new CheetahBuffer();
            for (var i = 0; i < 32; i++)
            {
                privateKey.Add(5);
            }

            return privateKey;
        }

        [MonoPInvokeCallback(typeof(CheetahObject.CreatedListener))]
        private static void OnCreate(ref CheetahCommandMeta meta, ref CheetahObjectId objectid)
        {
            instance.objectField.text = "Created " + objectid.id + " " + objectid.roomOwner;
        }

        private void Update()
        {
            CheetahClient.Receive();
            LoggerGateway.CollectLogs();
        }


        private void OnDestroy()
        {
            // CheetahClient.SetCurrentClient(ClientA);
            // CheetahClient.DestroyClient();
            // CheetahClient.SetCurrentClient(ClientB);
            // CheetahClient.DestroyClient();
        }

        private void OnLongUpdate(long value)
        {
            longField.text = value.ToString();
        }

        private void OnEvent(object eventstructure)
        {
            var structure = (EventWithMessage) eventstructure;
            eventField.text = structure.Message;
        }
    }
}