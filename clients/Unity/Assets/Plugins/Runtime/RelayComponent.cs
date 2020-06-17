using Cheetach.Relay;
using UnityEngine;

namespace Runtime
{
    public class RelayComponent : MonoBehaviour
    {
        void Start()
        {
            // // var dv = new TestStruct();
            // // var client = new NativeClient();
            // // client.Event += delegate(TestStruct s) { Debug.Log("Invoke from server " + s.id); };
            // // client.Collect();
            //
            // LowLevelApi.CallbackDelegate callback = delegate(ref S2CCommand command)
            // {
            //     Debug.Log("Command from server type = " + command.commandType);
            //     Debug.Log("Command from server id = " + command.upload.id);
            //     Debug.Log("Command from server long_counters_count = " + command.upload.long_counters_count);
            //     
            //     Debug.Log("Command from server long_counters[0].field_id = " + command.upload.long_counters[0].field_id);
            //     Debug.Log("Command from server long_counters[0].value = " + command.upload.long_counters[0].value);
            //     
            //     Debug.Log("Command from server long_counters[1].field_id = " + command.upload.long_counters[1].field_id);
            //     Debug.Log("Command from server long_counters[1].value = " + command.upload.long_counters[1].value);
            //     
            // };
            // LowLevelApi.test(callback);


            // var text = GetComponent<Text>();
            // if (text != null)
            // {
            //     text.text = trace;
            // }
        }
    }
}