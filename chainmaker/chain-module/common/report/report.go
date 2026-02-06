/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package report

import (
	"os"
	"strings"

	"github.com/go-echarts/go-echarts/v2/charts"
	"github.com/go-echarts/go-echarts/v2/opts"
)

// Report Used to generate charts, This method can be used to produce beautiful diagrams for stress testing
func Report(title, subtitle string, xAxis interface{}, series ...Series) {
	// create a new bar instance
	bar := charts.NewBar()
	// set some global options like Title/Legend/ToolTip or anything else
	bar.SetGlobalOptions(charts.WithTitleOpts(opts.Title{
		Title:    title,
		Subtitle: subtitle,
	}))

	// Put data into instance
	//bar.SetXAxis([]string{"Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"}).
	//	AddSeries("Category A", generateBarItems()).
	//	AddSeries("Category B", generateBarItems())
	axis := bar.SetXAxis(xAxis)
	for i := range series {
		axis.AddSeries(series[i].Name, series[i].Data, series[i].Options...)
	}
	title = strings.ToLower(title)
	title = strings.Replace(title, " ", "_", -1)
	// Where the magic happens
	f, _ := os.Create(title + ".html")
	_ = bar.Render(f)
}

type Series struct {
	Name    string
	Data    []opts.BarData
	Options []charts.SeriesOpts
}
