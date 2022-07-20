using System;
using System.Collections.Generic;
using Cheetah.Platform;
using Cheetah.Statistics.Events.Sender;
using UnityEngine;
using SystemInfo = UnityEngine.Device.SystemInfo;

namespace Cheetah.Statistics.Events
{
    public class StatisticsSession
    {
        private ISender sender;
        private string session = Guid.NewGuid().ToString();

        public StatisticsSession(ClusterConnector connector) : this(new GRPCSender(connector))
        {
        }

        public StatisticsSession(ISender sender)
        {
            this.sender = sender;
            Application.quitting += CloseSession;
            OpenSession(sender);
        }

        private void CloseSession()
        {
            sender.Send("Close session " + session, new Dictionary<string, string>()
            {
                ["close_session"] = "true",
                ["session"] = session
            });
        }

        private void OpenSession(ISender sender)
        {
            sender.Send("Open session " + session, new Dictionary<string, string>
            {
                ["open_session"] = "true",
                ["device_id"] = SystemInfo.deviceUniqueIdentifier,
                ["device_name"] = SystemInfo.deviceName,
                ["device_model"] = SystemInfo.deviceModel,
                ["processor_type"] = SystemInfo.processorType,
                ["processor_count"] = SystemInfo.processorCount.ToString(),
                ["processor_frequency"] = SystemInfo.processorFrequency.ToString(),
                ["battery_status"] = SystemInfo.batteryStatus.ToString(),
                ["battery_level"] = SystemInfo.batteryLevel.ToString(),
                ["graphics_device_model"] = SystemInfo.graphicsDeviceName,
                ["graphics_device_vendor"] = SystemInfo.graphicsDeviceVendor,
                ["graphics_device_version"] = SystemInfo.graphicsDeviceVersion,
                ["operation_system"] = SystemInfo.operatingSystem,
                ["version"] = Application.version,
                ["build_guid"] = Application.buildGUID,
                ["session"] = session
            });
        }

        internal void Send(string value, Dictionary<string, string> labels)
        {
            labels["session"] = session;
            sender.Send(value, labels);
        }
    }
}