using CheetahRelay;
using UnityEngine;
using UnityEngine.UI;

public class RelayTestComponent : MonoBehaviour
{
    [SerializeField] private Text text;

    void Start()
    {
        text.text = "Creating";
        var userPrivateKey = new CheetahBuffer();
        userPrivateKey.Add(10);
        var client = CheetahClient.CreateClient("127.0.0.1:8080", 10, 1, ref userPrivateKey, 0, out var clientId);
        text.text = "Created";
    }

    void Update()
    {
    }
}