using System.Collections.Generic;
using Cheetah.Statistics.Events.Sender;

namespace Cheetah.Matches.Statistics.Events.Test.Tests.Runtime
{
    public class StubSender : ISender
    {
        public List<SendItem> items = new();

        public struct SendItem
        {
            public string value;
            public Dictionary<string, string> labels;
        }

        public void Send(string value, Dictionary<string, string> labels)
        {
            items.Add(new SendItem()
            {
                value = value,
                labels = labels
            });
        }
    }
}