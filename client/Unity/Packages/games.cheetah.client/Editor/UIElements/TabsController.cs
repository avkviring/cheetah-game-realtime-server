using System;
using System.Collections.Generic;
using System.Diagnostics.Tracing;
using System.Linq;
using UnityEditor.UIElements;
using UnityEngine.UIElements;

namespace Games.Cheetah.Client.Editor.UIElements
{
    /// <summary>
    /// Управления переключением Tab
    /// </summary>
    public class TabsController
    {
        private IList<ToolbarToggle> tabs = new List<ToolbarToggle>();
        private Dictionary<ToolbarToggle, Action> tabToAction = new Dictionary<ToolbarToggle, Action>();

        public void RegisterTab(ToolbarToggle tab, Action action)
        {
            tabToAction[tab] = action;
            tabs.Add(tab);
            tab.RegisterCallback<ChangeEvent<bool>>(evt =>
            {
                if (!evt.newValue) return;
                if (!evt.newValue)
                {
                    tab.value = true;
                    return;
                }
                action.Invoke();
                foreach (var pair in tabToAction)
                {
                    var otherTab = pair.Key;
                    if (otherTab != tab)
                    {
                        otherTab.SetValueWithoutNotify(false);
                    }
                }
            });
        }

        public void SwitchToFirst()
        {
            var firstTab = tabs.First();
            firstTab.value = true;
            tabToAction[firstTab].Invoke();
            
        }
    }
}