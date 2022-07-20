using System.Collections.Generic;
using System.Linq;

namespace Cheetah.Platform.Editor.LocalServer
{
    /// <summary>
    /// Реестр серверных приложений
    /// </summary>
    public static class Registry
    {
        private static readonly IDictionary<string, IList<ServerApplication>> Applications = new Dictionary<string, IList<ServerApplication>>();

        private static readonly IDictionary<string, IList<IApplicationsConfigurator>> Configurators =
            new Dictionary<string, IList<IApplicationsConfigurator>>();

        public static void Register(string id, IList<ServerApplication> applications, IList<IApplicationsConfigurator> uis = null)
        {
            Applications[id] = applications;
            if (uis != null)
            {
                Configurators[id] = uis;
            }
            else
            {
                Configurators.Remove(id);
            }

            var allApplications = GetApplications();
            foreach (var serverApplication in allApplications)
            {
                serverApplication.ConfigureFromApplications(allApplications);
            }
        }

        public static List<IApplicationsConfigurator> GetConfigurators()
        {
            return Configurators
                .Values
                .SelectMany(c => c)
                .OrderByDescending(c => c.Order)
                .ThenBy(c => c.Title)
                .ToList();
        }

        public static List<ServerApplication> GetApplications()
        {
            return Applications
                .Values
                .SelectMany(applications => applications).ToList();
        }
    }
}