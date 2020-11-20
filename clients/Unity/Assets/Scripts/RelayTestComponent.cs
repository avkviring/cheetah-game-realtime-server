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

        void Start()
        {
        
            
        }

        void Update()
        {
            
            
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