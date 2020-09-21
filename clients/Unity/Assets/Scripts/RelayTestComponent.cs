using System.Collections.Generic;
using System.Globalization;
using CheetahRelay.Runtime.Client;
using CheetahRelay.Runtime.Client.Codec;
using MessagePack;
using MessagePack.Resolvers;
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
        private RelayClient clientA;
        private RelayClient clientB;
        private IRelayObject objectA1;
        
        void Start()
        {
        
            var messagePackCodecFactory = new MessagePackCodecFactory(GeneratedResolver.Instance);
            codecRegistry.RegisterEvent(0,messagePackCodecFactory.Create<EventWithMessage>() );
            codecRegistry.RegisterStructure(1, messagePackCodecFactory.Create<StructureWithMessage>() );

            clientA = new RelayClient("192.168.212.136:5000", "room-test-event", "client", codecRegistry);
            
            clientA.RegisterFactory(0, Create);
            objectA1 = clientA
                .GetObjectBuilder(0)
                .SetDoubleValue(1, 100.0)
                .SetLongValue(1, 100)
                .SetStructure(1, new StructureWithMessage {Message = "Structure"})
                .SetAccessGroup(5)
                .BuildAndSendToServer();

            clientB = new RelayClient("192.168.212.136:5000", "room-test-event", "client", codecRegistry);
            clientB.RegisterFactory(0, Create);
            eventField.text = "Starting test event";
            longField.text = "Starting test long";
        }

        void Update()
        {
            
                var eventWithMessage = new EventWithMessage {Message = Time.time.ToString(CultureInfo.InvariantCulture)};
                objectA1.SendEventToServer(0, eventWithMessage);
                objectA1.IncrementLongOnServer(0, 1);
                clientA.Update();
                clientB.Update();
        }

        private void Create(
            IRelayObject relayObject, 
            IReadOnlyDictionary<ushort, object> objectStructures,
            IReadOnlyDictionary<ushort, long> objectLongValues, 
            IReadOnlyDictionary<ushort, double> objectDoubleValues)
        {
            relayObject.SetEventListener(0, OnEvent);
            relayObject.SetLongUpdateListener(0, OnLongUpdate);
            objectField.text = ((StructureWithMessage) objectStructures[1]).Message + " " + objectLongValues[1] + " " + objectDoubleValues[1];
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