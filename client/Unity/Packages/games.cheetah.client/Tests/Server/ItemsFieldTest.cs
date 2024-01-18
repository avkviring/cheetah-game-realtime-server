using System.Collections.Generic;
using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using Games.Cheetah.Client.Tests.Server.Types;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.Client.Types.Object;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class ItemsFieldTest : AbstractTest
    {
        private TurretsParamsStructure turretsParamsA;
        private TurretsParamsStructure turretsParamsB;
        private NetworkObject networkObject;
        static FieldId.Items ItemsField = new(999);

        [SetUp]
        public void SetUp()
        {
            networkObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            turretsParamsA = new TurretsParamsStructure
            {
                Damage = 1.5,
                Speed = 154
            };
            turretsParamsB = new TurretsParamsStructure
            {
                Damage = 3,
                Speed = 200
            };
            clientA.Writer.AddItem(in networkObject.ObjectId, ItemsField, in turretsParamsA);
            clientA.Writer.AddItem(in networkObject.ObjectId, ItemsField, in turretsParamsB);
            Thread.Sleep(200);
            clientB.Update();
        }

        [Test]
        public void ShouldGetAddItemsWithNativeList()
        {
            var changes = clientB.Reader.GetAddedItems<TurretsParamsStructure>(ItemsField);
            Assert.AreEqual(changes[0].Item2.Damage, turretsParamsA.Damage);
            Assert.AreEqual(changes[1].Item2.Damage, turretsParamsB.Damage);
            changes.Dispose();
        }

        [Test]
        public void ShouldCollectAddItemst()
        {
            var changes = new List<(NetworkObjectId, TurretsParamsStructure)>();
            clientB.Reader.CollectAddedItems(ItemsField, changes);
            Assert.AreEqual(changes[0].Item2.Damage, turretsParamsA.Damage);
            Assert.AreEqual(changes[1].Item2.Damage, turretsParamsB.Damage);
        }
    }
}