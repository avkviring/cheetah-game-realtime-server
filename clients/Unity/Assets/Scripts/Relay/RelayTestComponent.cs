
using Cheetah.Matches.Relay.Command;
using UnityEngine;
using UnityEngine.UI;

public class RelayTestComponent : MonoBehaviour
{
    [SerializeField] private Text text;

    void Update()
    {
        text.text = "Creating";
        var userPrivateKey = new CheetahBuffer();
        userPrivateKey.Add(new byte[] {1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2});
        CheetahClient.CreateClient("127.0.0.1:5555", 1, 1, ref userPrivateKey, 0, out var clientId);
        CheetahClient.AttachToRoom();
        CheetahClient.DestroyClient();
        text.text = "Created";
    }
}