using System;
using MessagePack;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests.Runtime
{
    public class MessagePackTest
    {
        [Test]
        public void ClientTestSimplePasses()
        {
            // var client1 = new RelayClient("room-hash", "client-hash");
            // client1.Connect("127.0.0.1:5000");
            // var objectId1 = client1.CreateGameObject();
            //
            //
            // var client2 = new RelayClient("room-hash", "client-hash");
            // client2.Connect("127.0.0.1:5000");


            var someMessage = new SomeMessage {Age = 100500};
            var bytes = new byte[255];
            var byteArrayBufferWriter = new ByteArrayBufferWriter(bytes);
            MessagePackSerializer.Serialize(byteArrayBufferWriter, someMessage);
            Debug.Log("bytes " + bytes.Length);
            var someMessageDest = MessagePackSerializer.Deserialize<SomeMessage>(new ReadOnlyMemory<byte>(bytes, 0, byteArrayBufferWriter.Count));

            Assert.AreEqual(someMessage, someMessageDest);
        }
    }
}


[MessagePackObject]
public class SomeMessage
{
    protected bool Equals(SomeMessage other)
    {
        return Age == other.Age;
    }

    public override bool Equals(object obj)
    {
        if (ReferenceEquals(null, obj)) return false;
        if (ReferenceEquals(this, obj)) return true;
        if (obj.GetType() != this.GetType()) return false;
        return Equals((SomeMessage) obj);
    }

    public override int GetHashCode()
    {
        return Age.GetHashCode();
    }

    [Key(0)] public long Age { get; set; }
}