using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Cheetah.Platform;
using Cheetah.Statistics.Events.GRPC;
using UnityEngine.Assertions;

namespace Cheetah.Statistics.Events.Sender
{
    /// <summary>
    /// Отправка логов/событий на сервер через gRPC.
    /// </summary>
    internal class GRPCSender : ISender
    {
        private readonly ClusterConnector connector;

        public GRPCSender(ClusterConnector connector)
        {
            Assert.IsNotNull(connector, "connector == null");
            this.connector = connector;
        }


        public void Send(string value, Dictionary<string, string> labels)
        {
            var eventRequest = new EventRequest
            {
                Value = value,
                Time = (ulong)DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()
            };
            eventRequest.Labels.Add(labels);

            var action = new Action(async () =>
            {
                await connector.DoRequest(async channel =>
                {
                    var events = new GRPC.Events.EventsClient(channel);
                    await events.SendEventAsync(eventRequest);
                });
            });

            var task = new Task(action);
            task.Start();
        }
    }
}