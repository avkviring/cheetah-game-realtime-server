using System.Collections.Generic;
using System.Globalization;
using CheetahRelay.Runtime.Client;
using MessagePack;
using UnityEngine;
using UnityEngine.UI;
using MessagePackSerializer = CheetahRelay.Runtime.Client.Serialization.MessagePackSerializer;

/**
 * Тестирование основных возможностей RelayClient в сборке клиента
 */
public class RelayTestComponent : MonoBehaviour
{
    [SerializeField] public Text objectField;
    [SerializeField] public Text eventField;
    [SerializeField] public Text longField;

    private MessagePackSerializer messagePackSerializer = new MessagePackSerializer();
    private RelayClient clientA;
    private RelayClient clientB;
    private IRelayObject objectA1;


    [MessagePackObject]
    public class EventWithMessage
    {
        [Key(0)]
        public string Message { get; set; }
    }
    
    [MessagePackObject]
    public class StructureWithMessage
    {
        [Key(0)]
        public string Message { get; set; }
    }

    void Start()
    {
        
        messagePackSerializer.RegisterEvent<EventWithMessage>(0);
        messagePackSerializer.RegisterStructure<StructureWithMessage>(1);
        
        clientA = new RelayClient("192.168.212.136:5000", "room-test-event", "client", messagePackSerializer); ;
        clientA.RegisterFactory(0, Create);
        objectA1 = clientA
            .GetObjectBuilder(0)
            .SetDoubleValue(1, 100.0)
            .SetLongValue(1, 100)
            .SetStructure(1, new StructureWithMessage {Message = "Structure"})
            .SetAccessGroup(5)
            .BuildAndSendToServer();

        clientB = new RelayClient("192.168.212.136:5000", "room-test-event", "client", messagePackSerializer);
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

    private void Create(IRelayObject relayObject, IReadOnlyDictionary<ushort, object> objectStructures, IReadOnlyDictionary<ushort, long> objectLongValues, IReadOnlyDictionary<ushort, double> objectDoubleValues)
    {
        relayObject.SetEventListener(0, OnEvent);
        relayObject.SetLongUpdateListener(0, OnLongUpdate);
        objectField.text = ((StructureWithMessage) objectStructures[1]).Message + " " + objectLongValues[1] + " " + objectDoubleValues[1];
    }

    private void OnLongUpdate(long value)
    {
        longField.text = value.ToString();
    }

    private void OnEvent(object eventstructure) {
        var structure = (EventWithMessage) eventstructure;
        eventField.text = structure.Message;

    }
}