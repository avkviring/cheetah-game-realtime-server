using Cheetach.Relay;
using UnityEngine;

namespace Runtime
{
    public class RelayComponent : MonoBehaviour
    {
        void Start()
        {
            var dv = new TestStruct();
            var client = new NativeClient();
            client.Event += delegate(TestStruct s) { Debug.Log("Invoke from server " + s.id); };
            client.Collect();


            // var text = GetComponent<Text>();
            // if (text != null)
            // {
            //     text.text = trace;
            // }
        }
    }
}